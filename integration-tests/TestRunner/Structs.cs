using Xunit;

namespace TestRunner
{
    public class Structs
    {
        [Fact]
        public void CopyTupleStruct_RoundTrip()
        {
            var tuple = new CopyTupleStruct(1, 2);
            Assert.Equal(1, tuple.Element0);
            Assert.Equal(2, tuple.Element1);

            var result = IntegrationTests.CopyTupleStructRoundTrip(tuple);
            Assert.Equal(tuple, result);
            Assert.Equal(tuple.Element0, result.Element0);
            Assert.Equal(tuple.Element1, result.Element1);
        }
    }
}
