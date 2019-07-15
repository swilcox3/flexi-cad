var math = require("./math")

test('get join point index', () => {
    var points = [ new math.Point2d(0, 0), new math.Point2d(10, 0)]
    var join1 = new math.Point2d(0,0)
    expect(math.getJoinPtIndex(points, join1)).toEqual({success:true, pointIndex:0})
    var join2 = new math.Point2d(10, 0)
    expect(math.getJoinPtIndex(points, join2)).toEqual({success:true, pointIndex:1})
    var join3 = new math.Point2d(5, 0)
    expect(math.getJoinPtIndex(points, join3)).toEqual({success:true, pointIndex:1})
    expect(points.length).toEqual(3)
    var join4 = new math.Point2d(0, 5)
    expect(math.getJoinPtIndex(points, join4)).toEqual({success:false, pointIndex:0})
})
